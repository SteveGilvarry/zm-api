import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsUpdateManyMutationInput } from './control-presets-update-many-mutation.input';
import { ControlPresetsWhereInput } from './control-presets-where.input';

@ArgsType()
export class UpdateManyControlPresetsArgs {

    @Field(() => ControlPresetsUpdateManyMutationInput, {nullable:false})
    data!: ControlPresetsUpdateManyMutationInput;

    @Field(() => ControlPresetsWhereInput, {nullable:true})
    where?: ControlPresetsWhereInput;
}
