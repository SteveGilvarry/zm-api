import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';
import { Type } from 'class-transformer';
import { ControlsCreateInput } from './controls-create.input';
import { ControlsUpdateInput } from './controls-update.input';

@ArgsType()
export class UpsertOneControlsArgs {

    @Field(() => ControlsWhereUniqueInput, {nullable:false})
    @Type(() => ControlsWhereUniqueInput)
    where!: ControlsWhereUniqueInput;

    @Field(() => ControlsCreateInput, {nullable:false})
    @Type(() => ControlsCreateInput)
    create!: ControlsCreateInput;

    @Field(() => ControlsUpdateInput, {nullable:false})
    @Type(() => ControlsUpdateInput)
    update!: ControlsUpdateInput;
}
