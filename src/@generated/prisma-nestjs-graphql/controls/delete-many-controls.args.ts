import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsWhereInput } from './controls-where.input';

@ArgsType()
export class DeleteManyControlsArgs {

    @Field(() => ControlsWhereInput, {nullable:true})
    where?: ControlsWhereInput;
}
