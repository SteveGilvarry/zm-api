import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { ControlsService } from './controls.service';
import { ControlsCreateInput} from '../@generated/prisma-nestjs-graphql/controls/controls-create.input';
import { ControlsUpdateInput} from '../@generated/prisma-nestjs-graphql/controls/controls-update.input';
import { ControlsWhereUniqueInput } from '../@generated/prisma-nestjs-graphql/controls/controls-where-unique.input';

@Resolver('Control')
export class ControlsResolver {
  constructor(private readonly controlsService: ControlsService) {}

  @Mutation('createControl')
  async create(
    @Args('createControlInput') createControlInput: ControlsCreateInput,
    ) {
    const created = await this.controlsService.create(createControlInput);
  }

  @Query('controls')
  findAll() {
    return this.controlsService.findAll();
  }

  @Query('control')
  findOne(@Args('id') Id: number) {
    return this.controlsService.findOne({ Id });
  }

  @Mutation('updateControl')
  update(
    @Args( 'id') Id: number,
    @Args('updateControlInput') updateControlInput: ControlsUpdateInput) {
    return this.controlsService.update( { Id }, updateControlInput);
  }

  @Mutation('removeControl')
  remove(@Args('id') Id: number) {
    return this.controlsService.remove({ Id } );
  }
}
