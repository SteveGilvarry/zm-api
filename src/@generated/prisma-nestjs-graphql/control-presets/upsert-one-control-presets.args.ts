import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';
import { Type } from 'class-transformer';
import { ControlPresetsCreateInput } from './control-presets-create.input';
import { ControlPresetsUpdateInput } from './control-presets-update.input';

@ArgsType()
export class UpsertOneControlPresetsArgs {

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:false})
    @Type(() => ControlPresetsWhereUniqueInput)
    where!: ControlPresetsWhereUniqueInput;

    @Field(() => ControlPresetsCreateInput, {nullable:false})
    @Type(() => ControlPresetsCreateInput)
    create!: ControlPresetsCreateInput;

    @Field(() => ControlPresetsUpdateInput, {nullable:false})
    @Type(() => ControlPresetsUpdateInput)
    update!: ControlPresetsUpdateInput;
}
