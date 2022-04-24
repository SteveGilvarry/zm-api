import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsUpdateInput } from './control-presets-update.input';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';

@ArgsType()
export class UpdateOneControlPresetsArgs {

    @Field(() => ControlPresetsUpdateInput, {nullable:false})
    data!: ControlPresetsUpdateInput;

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:false})
    where!: ControlPresetsWhereUniqueInput;
}
