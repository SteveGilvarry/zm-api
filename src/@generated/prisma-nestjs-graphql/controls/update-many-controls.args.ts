import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsUpdateManyMutationInput } from './controls-update-many-mutation.input';
import { Type } from 'class-transformer';
import { ControlsWhereInput } from './controls-where.input';

@ArgsType()
export class UpdateManyControlsArgs {

    @Field(() => ControlsUpdateManyMutationInput, {nullable:false})
    @Type(() => ControlsUpdateManyMutationInput)
    data!: ControlsUpdateManyMutationInput;

    @Field(() => ControlsWhereInput, {nullable:true})
    @Type(() => ControlsWhereInput)
    where?: ControlsWhereInput;
}
