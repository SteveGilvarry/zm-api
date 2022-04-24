import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';
import { ControlPresetsCreateInput } from './control-presets-create.input';
import { ControlPresetsUpdateInput } from './control-presets-update.input';

@ArgsType()
export class UpsertOneControlPresetsArgs {

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:false})
    where!: ControlPresetsWhereUniqueInput;

    @Field(() => ControlPresetsCreateInput, {nullable:false})
    create!: ControlPresetsCreateInput;

    @Field(() => ControlPresetsUpdateInput, {nullable:false})
    update!: ControlPresetsUpdateInput;
}
