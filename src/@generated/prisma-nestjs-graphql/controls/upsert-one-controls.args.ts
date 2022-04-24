import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';
import { ControlsCreateInput } from './controls-create.input';
import { ControlsUpdateInput } from './controls-update.input';

@ArgsType()
export class UpsertOneControlsArgs {

    @Field(() => ControlsWhereUniqueInput, {nullable:false})
    where!: ControlsWhereUniqueInput;

    @Field(() => ControlsCreateInput, {nullable:false})
    create!: ControlsCreateInput;

    @Field(() => ControlsUpdateInput, {nullable:false})
    update!: ControlsUpdateInput;
}
