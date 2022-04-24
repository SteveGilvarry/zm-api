import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsUpdateManyMutationInput } from './controls-update-many-mutation.input';
import { ControlsWhereInput } from './controls-where.input';

@ArgsType()
export class UpdateManyControlsArgs {

    @Field(() => ControlsUpdateManyMutationInput, {nullable:false})
    data!: ControlsUpdateManyMutationInput;

    @Field(() => ControlsWhereInput, {nullable:true})
    where?: ControlsWhereInput;
}
