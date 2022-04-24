import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { FramesService } from './frames.service';
import { FramesCreateInput} from '../@generated/prisma-nestjs-graphql/frames/frames-create.input';
import { FramesUpdateInput} from '../@generated/prisma-nestjs-graphql/frames/frames-update.input';
import { FramesWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/frames/frames-where-unique.input';

@Resolver('Frame')
export class FramesResolver {
  constructor(private readonly framesService: FramesService) {}

  @Mutation('createFrame')
  async create(
    @Args('createFrameInput') createFrameInput: FramesCreateInput,
    ) {
    const created = await this.framesService.create(createFrameInput);
  }

  @Query('frames')
  findAll() {
    return this.framesService.findAll();
  }

  @Query('frame')
  findOne(@Args('id') Id: number) {
    return this.framesService.findOne({Id});
  }

  @Mutation('updateFrame')
  update(
    @Args('id') Id: number,
    @Args('updateFrameInput') updateFrameInput: FramesUpdateInput) {
    return this.framesService.update( {Id}, updateFrameInput);
  }

  @Mutation('removeFrame')
  remove(@Args('id') Id: number) {
    return this.framesService.remove({Id});
  }
}
